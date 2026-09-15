<?php

namespace Psalm\Plugin;

interface PluginEntryPointInterface {}

namespace Psalm\Plugin\EventHandler;

interface AfterMethodCallAnalysisInterface {}

namespace Psalm;

class PluginRegistrationSocket
{
    public function registerHooksFromClass(string $class): void {}
}

namespace Psalm\Plugin\EventHandler\Event;

class AfterMethodCallAnalysisEvent
{
    public function getAppearingMethodId(): string {}
    public function getExpr(): object {}
    public function getContext(): object {}
    public function getStatementsSource(): object {}
    public function setReturnTypeCandidate(mixed $type): void {}
}

namespace Psalm\Internal\Analyzer\Statements\Expression;

class ExpressionIdentifier
{
    public static function getExtendedVarId(mixed $expr, mixed $self, mixed $source): ?string {}
}
