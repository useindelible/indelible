package app.indelible.core.network

import kotlinx.io.IOException

internal val GATEWAY_GONE = 502..504

fun Throwable.gaveNoAnswer(): Boolean = this is IOException || (this is ApiException && statusCode in GATEWAY_GONE)
