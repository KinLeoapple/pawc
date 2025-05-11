package org.pawscript.config

data class PawConfig(
    val name: String? = null,
    val version: String? = null,
    val src: String? = null,
    val std: String? = null,
    val description: String? = null,
    val license: String? = null,
    val dependencies: List<String>? = null
)
