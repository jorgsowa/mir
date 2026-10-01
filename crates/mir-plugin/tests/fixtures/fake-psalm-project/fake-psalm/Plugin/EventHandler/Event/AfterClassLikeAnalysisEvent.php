<?php
namespace Psalm\Plugin\EventHandler\Event;

use Psalm\Codebase;
use Psalm\StatementsSource;
use Psalm\Storage\ClassLikeStorage;

final class AfterClassLikeAnalysisEvent
{
    public function __construct(
        private \PhpParser\Node\Stmt\ClassLike $stmt,
        private ClassLikeStorage $classlike_storage,
        private StatementsSource $statements_source,
        private Codebase $codebase,
        private array $file_replacements = []
    ) {
    }

    public function getStmt(): \PhpParser\Node\Stmt\ClassLike
    {
        return $this->stmt;
    }

    public function getClasslikeStorage(): ClassLikeStorage
    {
        return $this->classlike_storage;
    }

    public function getStatementsSource(): StatementsSource
    {
        return $this->statements_source;
    }

    public function getCodebase(): Codebase
    {
        return $this->codebase;
    }
}
