<?php
namespace Psalm\Internal;

use Psalm\Plugin\EventHandler\AfterClassLikeAnalysisInterface;
use Psalm\Plugin\EventHandler\AfterCodebasePopulatedInterface;
use Psalm\Plugin\EventHandler\Event\AfterClassLikeAnalysisEvent;
use Psalm\Plugin\EventHandler\Event\AfterCodebasePopulatedEvent;

class EventDispatcher
{
    /** @var list<class-string> */
    private array $handlers = [];

    public function registerClass(string $class): void
    {
        $this->handlers[] = $class;
    }

    public function dispatchAfterClassLikeAnalysis(AfterClassLikeAnalysisEvent $event): ?bool
    {
        foreach ($this->handlers as $handler) {
            if (is_subclass_of($handler, AfterClassLikeAnalysisInterface::class)) {
                $handler::afterStatementAnalysis($event);
            }
        }
        return null;
    }

    public function dispatchAfterCodebasePopulated(AfterCodebasePopulatedEvent $event): void
    {
        foreach ($this->handlers as $handler) {
            if (is_subclass_of($handler, AfterCodebasePopulatedInterface::class)) {
                $handler::afterCodebasePopulated($event);
            }
        }
    }
}
