<?php
namespace Psalm\Plugin\EventHandler;

use Psalm\Plugin\EventHandler\Event\MethodReturnTypeProviderEvent;
use Psalm\Type\Union;

interface MethodReturnTypeProviderInterface
{
    /** @return array<string> */
    public static function getClassLikeNames(): array;
    public static function getMethodReturnType(MethodReturnTypeProviderEvent $event): ?Union;
}
