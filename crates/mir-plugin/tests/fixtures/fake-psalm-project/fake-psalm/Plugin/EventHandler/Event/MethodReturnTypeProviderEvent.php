<?php
namespace Psalm\Plugin\EventHandler\Event;

use Psalm\CodeLocation;
use Psalm\Context;
use Psalm\StatementsSource;

final class MethodReturnTypeProviderEvent
{
    public function __construct(
        private StatementsSource $source,
        private string $fq_classlike_name,
        private string $method_name_lowercase,
        private \PhpParser\Node\Expr\MethodCall $stmt,
        private Context $context,
        private CodeLocation $code_location,
        private ?array $template_type_parameters = null,
        private ?string $called_fq_classlike_name = null,
        private ?string $called_method_name_lowercase = null
    ) {
    }

    public function getSource(): StatementsSource
    {
        return $this->source;
    }

    public function getFqClasslikeName(): string
    {
        return $this->fq_classlike_name;
    }

    public function getMethodNameLowercase(): string
    {
        return $this->method_name_lowercase;
    }

    public function getCallArgs(): array
    {
        return $this->stmt->args;
    }

    public function getCodeLocation(): CodeLocation
    {
        return $this->code_location;
    }
}
