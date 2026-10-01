<?php
namespace Psalm\Plugin\EventHandler\Event;

use Psalm\Codebase;

final class AfterCodebasePopulatedEvent
{
    public function __construct(private Codebase $codebase)
    {
    }

    public function getCodebase(): Codebase
    {
        return $this->codebase;
    }
}
