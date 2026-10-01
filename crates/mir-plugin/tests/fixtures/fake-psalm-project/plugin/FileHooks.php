<?php
namespace TestPlugin;

use Psalm\IssueBuffer;
use Psalm\Plugin\EventHandler\AfterFunctionLikeAnalysisInterface;
use Psalm\Plugin\EventHandler\Event\AfterFunctionLikeAnalysisEvent;
use Psalm\Plugin\EventHandler\Event\MethodReturnTypeProviderEvent;
use Psalm\Plugin\EventHandler\MethodReturnTypeProviderInterface;
use Psalm\Type;
use Psalm\Type\Union;

// Loaded as a file-based plugin: the first class declared here is the hook class.
final class FileHooks implements MethodReturnTypeProviderInterface, AfterFunctionLikeAnalysisInterface
{
    public static function getClassLikeNames(): array
    {
        return ['App\\Registry'];
    }

    public static function getMethodReturnType(MethodReturnTypeProviderEvent $event): ?Union
    {
        if ($event->getSource()->getFQCLN() === null) {
            IssueBuffer::maybeAdd(new TestIssue('called outside a class', $event->getCodeLocation()));
            return null;
        }

        return Type::parseString('string');
    }

    public static function afterStatementAnalysis(AfterFunctionLikeAnalysisEvent $event): ?bool
    {
        $storage = $event->getFunctionlikeStorage();
        foreach ($storage->params as $param) {
            foreach ($param->getAttributeStorages() as $attribute) {
                $declared = $param->type === null ? 'none' : (string)$param->type;
                $key = (string)$attribute->args[0]->type;
                IssueBuffer::maybeAdd(new TestIssue(
                    "{$storage->cased_name}: \${$param->name} {$declared} {$attribute->fq_class_name} {$key}",
                    $attribute->location
                ));
            }
        }
        return null;
    }
}
