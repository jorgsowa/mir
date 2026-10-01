<?php
namespace Psalm\Plugin\EventHandler\Event;

use Psalm\Codebase;
use Psalm\Context;
use Psalm\NodeTypeProvider;
use Psalm\StatementsSource;
use Psalm\Storage\FunctionLikeStorage;

final class AfterFunctionLikeAnalysisEvent
{
    public function __construct(
        private \PhpParser\Node\FunctionLike $stmt,
        private FunctionLikeStorage $functionlike_storage,
        private StatementsSource $statements_source,
        private Codebase $codebase,
        private array $file_replacements,
        private NodeTypeProvider $node_type_provider,
        private Context $context
    ) {
    }

    public function getFunctionlikeStorage(): FunctionLikeStorage
    {
        return $this->functionlike_storage;
    }

    public function getStatementsSource(): StatementsSource
    {
        return $this->statements_source;
    }

    public function getStmt(): \PhpParser\Node\FunctionLike
    {
        return $this->stmt;
    }
}
