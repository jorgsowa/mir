<?php
namespace Psalm\Storage;

class ClassLikeStorage
{
    /** @var list<string> */
    public array $suppressed_issues = [];
    public ?object $aliases = null;
    /** @var array<string, true> */
    public array $methods = [];

    public function __construct(public string $name)
    {
    }
}
