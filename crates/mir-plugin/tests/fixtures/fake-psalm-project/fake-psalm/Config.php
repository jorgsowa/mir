<?php
namespace Psalm;

class Config
{
    public static function loadFromXML(string $base_dir, string $xml): self
    {
        return new self();
    }
}
