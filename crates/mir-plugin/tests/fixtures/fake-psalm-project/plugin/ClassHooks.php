<?php
namespace TestPlugin;

use Psalm\CodeLocation\Raw;
use Psalm\IssueBuffer;
use Psalm\Plugin\EventHandler\AfterClassLikeAnalysisInterface;
use Psalm\Plugin\EventHandler\AfterCodebasePopulatedInterface;
use Psalm\Plugin\EventHandler\Event\AfterClassLikeAnalysisEvent;
use Psalm\Plugin\EventHandler\Event\AfterCodebasePopulatedEvent;

// Loaded as a file-based plugin: the first class declared here is the hook class.
final class ClassHooks implements AfterClassLikeAnalysisInterface, AfterCodebasePopulatedInterface
{
    private static int $populated = 0;

    public static function afterCodebasePopulated(AfterCodebasePopulatedEvent $event)
    {
        self::$populated++;
    }

    public static function afterStatementAnalysis(AfterClassLikeAnalysisEvent $event): ?bool
    {
        $storage = $event->getClasslikeStorage();
        $file = $event->getStatementsSource()->getFilePath();
        $codebase = $event->getCodebase();

        $storage->suppressed_issues[] = 'MissingConstructor';
        $storage->methods['helper'] = true;
        $codebase->file_reference_provider->addNonMethodReferenceToClass($file, strtolower($storage->name));
        $codebase->file_reference_provider->addMethodReferenceToClassMember(
            'phpunit\\framework\\testsuite::run',
            strtolower($storage->name) . '::helper'
        );
        IssueBuffer::maybeAdd(new TestIssue(
            'class ' . $storage->name . ' populated=' . self::$populated . ' scanned=' . implode(',', array_map('basename', $codebase->scanned)),
            new Raw('', $file, basename($file), 5, 9)
        ));
        return null;
    }
}
