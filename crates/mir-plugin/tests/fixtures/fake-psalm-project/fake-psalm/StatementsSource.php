<?php
namespace Psalm;

interface StatementsSource
{
    public function getNodeTypeProvider(): NodeTypeProvider;
    public function getFilePath(): string;
    public function getFileName(): string;
    public function getAliases(): Aliases;
    /** @return list<string> */
    public function getSuppressedIssues(): array;
    public function getCodebase(): Codebase;
    public function getFQCLN(): ?string;
}
