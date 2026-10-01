<?php
namespace Psalm\Plugin\EventHandler;

use Psalm\Plugin\EventHandler\Event\AfterCodebasePopulatedEvent;

interface AfterCodebasePopulatedInterface
{
    public static function afterCodebasePopulated(AfterCodebasePopulatedEvent $event);
}
