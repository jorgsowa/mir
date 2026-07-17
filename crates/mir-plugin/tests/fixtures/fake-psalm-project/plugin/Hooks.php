<?php
namespace TestPlugin;

use Psalm\Plugin\EventHandler\AfterExpressionAnalysisInterface;
use Psalm\Plugin\EventHandler\Event\FunctionReturnTypeProviderEvent;
use Psalm\Plugin\EventHandler\FunctionReturnTypeProviderInterface;
use Psalm\Type;
use Psalm\Type\Union;

class Hooks implements FunctionReturnTypeProviderInterface, AfterExpressionAnalysisInterface
{
    public static function getFunctionIds(): array
    {
        return ['test_helper'];
    }

    public static function getFunctionReturnType(FunctionReturnTypeProviderEvent $event): ?Union
    {
        $args = $event->getCallArgs();
        if ($args === []) {
            return null;
        }
        $argType = $event->getStatementsSource()
            ->getNodeTypeProvider()
            ->getType($args[0]->value);

        return Type::parseString('list<' . (string)$argType . '>');
    }

    public static function afterExpressionAnalysis(object $event): ?bool
    {
        return null;
    }
}
