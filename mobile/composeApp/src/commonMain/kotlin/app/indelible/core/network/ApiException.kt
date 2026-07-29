package app.indelible.core.network

class ApiException(
    val statusCode: Int,
    override val message: String,
) : Exception(message)
