<?php
namespace Psalm\Plugin\EventHandler;

use Psalm\Plugin\EventHandler\Event\AfterClassLikeAnalysisEvent;

interface AfterClassLikeAnalysisInterface
{
    public static function afterStatementAnalysis(AfterClassLikeAnalysisEvent $event): ?bool;
}
