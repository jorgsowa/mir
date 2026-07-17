<?php
namespace Psalm\CodeLocation;

use Psalm\CodeLocation;

class Raw extends CodeLocation
{
    public function __construct(
        public string $file_contents,
        public string $file_path,
        public string $file_name,
        public int $file_start,
        public int $file_end
    ) {
    }
}
