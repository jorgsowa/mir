<?php
namespace TestPlugin;

use Psalm\Plugin\PluginEntryPointInterface;
use Psalm\Plugin\RegistrationInterface;

class Plugin implements PluginEntryPointInterface
{
    public function __invoke(RegistrationInterface $registration, ?\SimpleXMLElement $config = null): void
    {
        $registration->addStubFile(__DIR__ . '/stubs/helpers.phpstub');
        $registration->registerHooksFromClass(Hooks::class);
    }
}
