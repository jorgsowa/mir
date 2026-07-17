<?php
namespace Psalm\Plugin;

interface PluginEntryPointInterface
{
    public function __invoke(RegistrationInterface $registration, ?\SimpleXMLElement $config = null): void;
}
