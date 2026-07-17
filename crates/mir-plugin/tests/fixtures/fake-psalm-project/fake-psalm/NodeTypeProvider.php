<?php
namespace Psalm;

use Psalm\Type\Union;

interface NodeTypeProvider
{
    public function getType(object $node): ?Union;
    public function setType(object $node, Union $type): void;
}
