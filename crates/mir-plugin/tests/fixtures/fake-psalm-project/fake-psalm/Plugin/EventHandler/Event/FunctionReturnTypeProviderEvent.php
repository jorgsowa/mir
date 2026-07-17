<?php
namespace Psalm\Plugin\EventHandler\Event;

use Psalm\CodeLocation;
use Psalm\Context;
use Psalm\StatementsSource;

final class FunctionReturnTypeProviderEvent
{
    public function __construct(
        private StatementsSource $statements_source,
        private string $function_id,
        private array $call_args,
        private Context $context,
        private CodeLocation $code_location
    ) {
    }

    public function getStatementsSource(): StatementsSource
    {
        return $this->statements_source;
    }

    public function getFunctionId(): string
    {
        return $this->function_id;
    }

    public function getCallArgs(): array
    {
        return $this->call_args;
    }
}
