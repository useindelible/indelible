use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::{
    SetLocaleOverrideParams, SetUserAgentOverrideParams, UserAgentBrandVersion, UserAgentMetadata,
};

use crate::config::CaptureSettings;
use crate::render::{CaptureError, CaptureStage};

const BROWSER_INIT_SCRIPT: &str = r#"
(() => {
  Object.defineProperty(Object.getPrototypeOf(navigator), 'webdriver', {
    configurable: true,
    get: () => false,
  });

  const plugin = Object.freeze({
    name: 'PDF Viewer',
    filename: 'internal-pdf-viewer',
    description: 'Portable Document Format',
    length: 1,
    0: Object.freeze({ type: 'application/pdf', suffixes: 'pdf', description: 'Portable Document Format' }),
  });
  const plugins = Object.create(PluginArray.prototype);
  Object.defineProperties(plugins, {
    0: { value: plugin, enumerable: true },
    length: { value: 1 },
    item: { value: index => index === 0 ? plugin : null },
    namedItem: { value: name => name === plugin.name ? plugin : null },
    refresh: { value: () => undefined },
    [Symbol.iterator]: { value: function* () { yield plugin; } },
  });
  Object.defineProperty(Object.getPrototypeOf(navigator), 'plugins', {
    configurable: true,
    get: () => plugins,
  });
})();
"#;

pub(crate) async fn apply_live_browser_identity(
    page: &Page,
    settings: &CaptureSettings,
) -> Result<(), CaptureError> {
    let raw_user_agent = page
        .user_agent()
        .await
        .map_err(|error| CaptureError::cdp(CaptureStage::Identity, error))?;
    let (user_agent, full_version, major_version) = normalize_user_agent(&raw_user_agent)?;
    let metadata = user_agent_metadata(&raw_user_agent, &full_version, &major_version)?;
    let params = SetUserAgentOverrideParams::builder()
        .user_agent(user_agent)
        .accept_language(format!("{},en;q=0.9", settings.locale))
        .platform(navigator_platform(&raw_user_agent))
        .user_agent_metadata(metadata)
        .build()
        .map_err(|error| CaptureError::other(CaptureStage::Identity, anyhow::anyhow!(error)))?;

    page.execute(params)
        .await
        .map_err(|error| CaptureError::cdp(CaptureStage::Identity, error))?;
    page.execute(
        SetLocaleOverrideParams::builder()
            .locale(settings.locale.clone())
            .build(),
    )
    .await
    .map_err(|error| CaptureError::cdp(CaptureStage::Identity, error))?;
    page.emulate_timezone(settings.timezone.clone())
        .await
        .map_err(|error| CaptureError::cdp(CaptureStage::Identity, error))?;
    page.add_init_script(BROWSER_INIT_SCRIPT)
        .await
        .map_err(|error| CaptureError::cdp(CaptureStage::Identity, error))?;
    Ok(())
}

fn normalize_user_agent(raw: &str) -> Result<(String, String, String), CaptureError> {
    let normalized = raw.replace("HeadlessChrome/", "Chrome/");
    let full_version = normalized
        .split_whitespace()
        .find_map(|token| token.strip_prefix("Chrome/"))
        .ok_or_else(|| {
            CaptureError::other(
                CaptureStage::Identity,
                anyhow::anyhow!("installed browser UA contains no Chrome version: {raw}"),
            )
        })?
        .to_string();
    let major_version = full_version
        .split('.')
        .next()
        .unwrap_or(&full_version)
        .to_string();
    Ok((normalized, full_version, major_version))
}

fn user_agent_metadata(
    raw_user_agent: &str,
    full_version: &str,
    major_version: &str,
) -> Result<UserAgentMetadata, CaptureError> {
    let (brands, full_version_list) = client_hint_brand_lists(full_version, major_version)?;

    UserAgentMetadata::builder()
        .brands(brands)
        .full_version_lists(full_version_list)
        .platform(client_hint_platform(raw_user_agent))
        .platform_version("")
        .architecture(client_hint_architecture())
        .model("")
        .mobile(false)
        .bitness("64")
        .build()
        .map_err(identity_build_error)
}

fn client_hint_brand_lists(
    full_version: &str,
    major_version: &str,
) -> Result<(Vec<UserAgentBrandVersion>, Vec<UserAgentBrandVersion>), CaptureError> {
    const GREASE_CHARACTERS: [&str; 11] = [" ", "(", ":", "-", ".", "/", ")", ";", "=", "?", "_"];
    const GREASE_VERSIONS: [&str; 3] = ["8", "99", "24"];

    let seed = major_version.parse::<usize>().map_err(|error| {
        CaptureError::other(
            CaptureStage::Identity,
            anyhow::anyhow!("invalid installed browser major version {major_version}: {error}"),
        )
    })?;
    let grease_brand = format!(
        "Not{}A{}Brand",
        GREASE_CHARACTERS[seed % GREASE_CHARACTERS.len()],
        GREASE_CHARACTERS[(seed + 1) % GREASE_CHARACTERS.len()]
    );
    let grease_version = GREASE_VERSIONS[seed % GREASE_VERSIONS.len()];

    let brands = shuffle_client_hint_brands(
        [
            UserAgentBrandVersion::new(grease_brand.clone(), grease_version),
            UserAgentBrandVersion::new("Chromium", major_version),
            UserAgentBrandVersion::new("Chrome", major_version),
        ],
        seed,
    );
    let full_version_list = shuffle_client_hint_brands(
        [
            UserAgentBrandVersion::new(grease_brand, format!("{grease_version}.0.0.0")),
            UserAgentBrandVersion::new("Chromium", full_version),
            UserAgentBrandVersion::new("Chrome", full_version),
        ],
        seed,
    );

    Ok((brands, full_version_list))
}

fn shuffle_client_hint_brands(
    input: [UserAgentBrandVersion; 3],
    seed: usize,
) -> Vec<UserAgentBrandVersion> {
    const ORDERS: [[usize; 3]; 6] = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];

    let mut output = [None, None, None];
    for (brand, output_index) in input.into_iter().zip(ORDERS[seed % ORDERS.len()]) {
        output[output_index] = Some(brand);
    }
    output.into_iter().flatten().collect()
}

fn identity_build_error(error: String) -> CaptureError {
    CaptureError::other(CaptureStage::Identity, anyhow::anyhow!(error))
}

fn navigator_platform(raw_user_agent: &str) -> &'static str {
    if raw_user_agent.contains("Macintosh") {
        "MacIntel"
    } else if raw_user_agent.contains("Windows") {
        "Win32"
    } else {
        "Linux x86_64"
    }
}

fn client_hint_platform(raw_user_agent: &str) -> &'static str {
    if raw_user_agent.contains("Macintosh") {
        "macOS"
    } else if raw_user_agent.contains("Windows") {
        "Windows"
    } else {
        "Linux"
    }
}

fn client_hint_architecture() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm",
        "x86_64" => "x86",
        architecture => architecture,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_identity_removes_headless_token_and_matches_client_hints() {
        let raw = "Mozilla/5.0 (X11; Linux x86_64) HeadlessChrome/148.0.7778.97 Safari/537.36";
        let (user_agent, full, major) = normalize_user_agent(raw).unwrap();
        assert!(!user_agent.contains("HeadlessChrome"));
        assert!(user_agent.contains("Chrome/148.0.7778.97"));
        let metadata = user_agent_metadata(&user_agent, &full, &major).unwrap();
        let brands = metadata.brands.unwrap();
        assert!(
            brands
                .iter()
                .any(|brand| brand.brand == "Chromium" && brand.version == "148")
        );
        assert!(
            metadata
                .full_version_list
                .unwrap()
                .iter()
                .any(|brand| { brand.brand == "Chromium" && brand.version == "148.0.7778.97" })
        );
    }
}
