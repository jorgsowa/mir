<?php
namespace Psalm\Plugin\EventHandler;

use Psalm\Plugin\EventHandler\Event\FunctionReturnTypeProviderEvent;
use Psalm\Type\Union;

interface FunctionReturnTypeProviderInterface
{
    /** @return array<lowercase-string> */
    public static function getFunctionIds(): array;
    public static function getFunctionReturnType(FunctionReturnTypeProviderEvent $event): ?Union;
}
