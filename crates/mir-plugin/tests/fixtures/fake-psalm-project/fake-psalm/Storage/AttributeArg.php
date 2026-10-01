<?php
namespace Psalm\Storage;

use Psalm\CodeLocation;
use Psalm\Type\Union;

final class AttributeArg
{
    public function __construct(
        public readonly ?string $name,
        public readonly Union $type,
        public readonly CodeLocation $location
    ) {
    }
}
