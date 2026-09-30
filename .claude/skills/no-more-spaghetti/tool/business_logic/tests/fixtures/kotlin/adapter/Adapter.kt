package adapter

import core.Layer

object Facade {
    fun label(): String = Layer.name()
}
