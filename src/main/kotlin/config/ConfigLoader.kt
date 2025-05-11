package org.pawscript.config

import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.dataformat.yaml.YAMLFactory
import com.fasterxml.jackson.dataformat.yaml.YAMLMapper
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
import com.fasterxml.jackson.module.kotlin.registerKotlinModule
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths

object ConfigLoader {
    private val yamlMapper = YAMLMapper(YAMLFactory()).apply {
        registerKotlinModule()
        configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
    }
    private val jsonMapper = jacksonObjectMapper().apply {
        registerKotlinModule()
        configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
    }

    /**
     * 在 projectRoot 下，读取 pawconfig.yaml 或 pawconfig.json
     * 如果都不存在，则返回一个空的 PawConfig（使用默认值）
     */
    fun load(projectRoot: Path): PawConfig {
        val yamlFile = projectRoot.resolve("pawconfig.yaml")
        val jsonFile = projectRoot.resolve("pawconfig.json")

        return when {
            Files.exists(yamlFile) -> {
                val text = Files.readString(yamlFile)
                yamlMapper.readValue<PawConfig>(text)
            }
            Files.exists(jsonFile) -> {
                val text = Files.readString(jsonFile)
                jsonMapper.readValue<PawConfig>(text)
            }
            else -> PawConfig()
        }
    }

    /**
     * projectRoot: 项目根目录
     * cfg.std: 配置里可选指定 std 路径（相对或绝对）
     *
     * 返回 Pair(srcDir, stdDir)
     */
    fun resolvePaths(projectRoot: Path, cfg: PawConfig): Pair<Path,Path> {
        // 1) 源码目录：始终基于 projectRoot
        val srcDir = projectRoot.resolve(cfg.src ?: "src")

        val stdDir = cfg.std?.let {
            val p = Paths.get(it)
            if (p.isAbsolute) p else projectRoot.resolve(p)
        } ?: System.getenv("PAW_STD")?.let { Paths.get(it) }
        ?: detectExecDir()?.resolve("stdlib")
        ?: error("Cannot locate standard library: please set pawconfig.std or PAW_STD")

        return srcDir to stdDir
    }

    /**
     * 获取当前进程可执行文件所在目录（支持 .exe、native image、jar等）
     */
    private fun detectExecDir(): Path? {
        return try {
            val cmd = ProcessHandle.current()
                .info()
                .command()
                .orElse(null) ?: return null
            Paths.get(cmd).parent
        } catch (_: Throwable) {
            null
        }
    }
}