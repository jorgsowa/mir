<?php
namespace Psalm\Storage;

use Psalm\CodeLocation;

final class AttributeStorage
{
    /** @param list<AttributeArg> $args */
    public function __construct(
        public readonly string $fq_class_name,
        public readonly array $args,
        public readonly CodeLocation $location,
        public readonly CodeLocation $name_location
    ) {
    }
}
