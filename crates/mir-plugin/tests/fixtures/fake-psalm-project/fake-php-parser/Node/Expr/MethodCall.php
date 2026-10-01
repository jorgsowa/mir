<?php
namespace PhpParser\Node\Expr;

class MethodCall
{
    public function __construct(public object $var, public object $name, public array $args = [])
    {
    }
}
