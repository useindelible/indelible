package app.indelible.i18ncheck

import org.w3c.dom.Element
import java.io.File
import javax.xml.XMLConstants
import javax.xml.parsers.DocumentBuilderFactory

data class CatalogResource(
    val type: String,
    val values: Map<String, String>,
)

data class CatalogReadResult(
    val resources: Map<String, CatalogResource>,
    val errors: List<String>,
)

class XmlCatalogReader(
    private val allowedPrefixes: Set<String>,
) {
    fun read(file: File): CatalogReadResult {
        val document = secureFactory().newDocumentBuilder().parse(file)
        val resources = linkedMapOf<String, CatalogResource>()
        val errors = mutableListOf<String>()
        document.documentElement.childElements().filter { it.tagName in RESOURCE_TAGS }.forEach { element ->
            val name = element.getAttribute("name")
            if (name.isBlank()) errors += "${file.path}: resource name must not be empty"
            if (name in resources) errors += "${file.path}: duplicate resource $name"
            if (allowedPrefixes.none(name::startsWith)) errors += "${file.path}: unsupported resource prefix for $name"

            val values =
                if (element.tagName == "string") {
                    linkedMapOf("value" to element.textContent.trim())
                } else {
                    readPluralValues(file, name, element, errors)
                }
            if (values.isEmpty() || values.values.any(String::isBlank)) {
                errors += "${file.path}: $name must not contain empty values"
            }
            resources[name] = CatalogResource(element.tagName, values)
        }

        val keys = resources.keys.toList()
        if (keys != keys.sorted()) {
            val mismatch = keys.zip(keys.sorted()).first { (actual, expected) -> actual != expected }
            errors +=
                "${file.path}: resources must be alphabetically sorted; " +
                "found ${mismatch.first} before ${mismatch.second}"
        }
        return CatalogReadResult(resources, errors)
    }

    private fun readPluralValues(
        file: File,
        name: String,
        element: Element,
        errors: MutableList<String>,
    ): Map<String, String> {
        val values = linkedMapOf<String, String>()
        element.childElements().filter { it.tagName == "item" }.forEach { item ->
            val quantity = item.getAttribute("quantity")
            if (quantity in values) errors += "${file.path}: duplicate $name quantity $quantity"
            values[quantity] = item.textContent.trim()
        }
        return values
    }

    private fun Element.childElements(): Sequence<Element> =
        (0 until childNodes.length).asSequence().mapNotNull { childNodes.item(it) as? Element }

    private fun secureFactory(): DocumentBuilderFactory =
        DocumentBuilderFactory.newInstance().apply {
            setFeature(XMLConstants.FEATURE_SECURE_PROCESSING, true)
            setFeature("http://apache.org/xml/features/disallow-doctype-decl", true)
            setFeature("http://xml.org/sax/features/external-general-entities", false)
            setFeature("http://xml.org/sax/features/external-parameter-entities", false)
            setFeature("http://apache.org/xml/features/nonvalidating/load-external-dtd", false)
            setAttribute(XMLConstants.ACCESS_EXTERNAL_DTD, "")
            setAttribute(XMLConstants.ACCESS_EXTERNAL_SCHEMA, "")
            isXIncludeAware = false
            isExpandEntityReferences = false
        }

    private companion object {
        val RESOURCE_TAGS = setOf("plurals", "string")
    }
}
