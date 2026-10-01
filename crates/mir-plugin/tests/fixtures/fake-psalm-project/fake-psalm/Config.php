<?php
namespace Psalm;

use Psalm\Internal\EventDispatcher;

class Config
{
    public EventDispatcher $eventDispatcher;

    public function __construct()
    {
        $this->eventDispatcher = new EventDispatcher();
    }

    public static function loadFromXML(string $base_dir, string $xml): self
    {
        return new self();
    }

    public function visitStubFiles(Codebase $codebase): void
    {
    }

    public function setComposerClassLoader(object $loader): void
    {
    }
}
