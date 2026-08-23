import UIKit
import SwiftUI
import MobileCoreServices
import UniformTypeIdentifiers
import ComposeApp

// The extension shares keychain items and NSUserDefaults with the main app via
// the "group.com.useindelible.app" App Group, so both targets must be signed by
// the same team. ComposeApp is a static framework: it is linked directly into
// this binary and the app's, so nothing is embedded or resolved at runtime.

class ShareViewController: UIViewController {
    private let bridge = ShareExtensionBridge()

    deinit {
        bridge.close()
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        extractURL { [weak self] url in
            guard let self, let url else {
                self?.cancel()
                return
            }
            self.presentSaveSheet(for: url)
        }
    }

    private func extractURL(completion: @escaping (String?) -> Void) {
        guard let item = extensionContext?.inputItems.first as? NSExtensionItem,
              let provider = item.attachments?.first else {
            completion(nil)
            return
        }

        if provider.hasItemConformingToTypeIdentifier(UTType.url.identifier) {
            provider.loadItem(forTypeIdentifier: UTType.url.identifier) { data, _ in
                DispatchQueue.main.async {
                    completion((data as? URL)?.absoluteString)
                }
            }
        } else if provider.hasItemConformingToTypeIdentifier(UTType.plainText.identifier) {
            provider.loadItem(forTypeIdentifier: UTType.plainText.identifier) { data, _ in
                DispatchQueue.main.async {
                    completion(data as? String)
                }
            }
        } else {
            completion(nil)
        }
    }

    private func presentSaveSheet(for url: String) {
        let model = ShareSheetModel()
        let saveView = SaveSheetView(
            url: url,
            model: model,
            onSave: { [weak self] in
                self?.bridge.save(url: url, completion: { success, message in
                    switch (success, message) {
                    case (true, "queued"):
                        model.state = .queued
                        self?.completeAfterShowingResult()
                    case (true, "already_saved"):
                        model.state = .alreadySaved
                        self?.completeAfterShowingResult()
                    case (true, _):
                        model.state = .saved
                        self?.completeAfterShowingResult()
                    case (_, "auth_required"):
                        model.state = .authRequired
                    case (_, "invalid_url"):
                        model.state = .invalidURL
                    default:
                        model.state = .error
                    }
                })
            },
            onCancel: { [weak self] in
                self?.cancel()
            },
            onSignIn: { [weak self] in
                self?.openMainAppForAuth()
            }
        )

        let host = UIHostingController(rootView: saveView)
        host.view.backgroundColor = .clear
        addChild(host)
        view.addSubview(host.view)
        host.view.translatesAutoresizingMaskIntoConstraints = false
        NSLayoutConstraint.activate([
            host.view.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            host.view.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            host.view.topAnchor.constraint(equalTo: view.topAnchor),
            host.view.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
        host.didMove(toParent: self)
    }

    private func completeAfterShowingResult() {
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.8) { [weak self] in
            self?.done()
        }
    }

    private func done() {
        extensionContext?.completeRequest(returningItems: nil)
    }

    private func cancel() {
        extensionContext?.cancelRequest(withError: NSError(domain: "IndelibleShareExtension", code: 0))
    }

    private func openMainAppForAuth() {
        if let url = URL(string: "indelible://auth/login") {
            extensionContext?.open(url)
        }
        cancel()
    }
}
