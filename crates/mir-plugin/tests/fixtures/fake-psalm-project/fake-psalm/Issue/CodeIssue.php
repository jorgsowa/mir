<?php
namespace Psalm\Issue;

use Psalm\CodeLocation;

abstract class CodeIssue
{
    public function __construct(public string $message, public CodeLocation $code_location)
    {
    }
}
