<?php
namespace Psalm\Storage;

abstract class FunctionLikeStorage
{
    public ?string $cased_name = null;
    public ?\Psalm\CodeLocation $location = null;
    /** @var list<FunctionLikeParameter> */
    public array $params = [];
}
