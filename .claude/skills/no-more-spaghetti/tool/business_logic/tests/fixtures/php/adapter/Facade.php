<?php
namespace adapter;

use core\Layer;

final class Facade
{
    public static function label(): string
    {
        return Layer::name();
    }
}
