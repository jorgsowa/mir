<?php
namespace Psalm\Plugin\EventHandler;

interface AfterExpressionAnalysisInterface
{
    public static function afterExpressionAnalysis(object $event): ?bool;
}
