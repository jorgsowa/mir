<?php
namespace Psalm\Storage;

use Psalm\Type\Union;

class FunctionLikeParameter
{
    /** @var list<AttributeStorage> */
    public array $attributes = [];

    public function __construct(
        public string $name,
        public bool $by_ref,
        public ?Union $type = null,
        public ?Union $signature_type = null
    ) {
    }

    /** @return list<AttributeStorage> */
    public function getAttributeStorages(): array
    {
        return $this->attributes;
    }
}
