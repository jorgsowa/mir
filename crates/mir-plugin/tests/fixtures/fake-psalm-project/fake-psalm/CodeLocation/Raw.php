<?php
namespace Psalm\CodeLocation;

use Psalm\CodeLocation;

class Raw extends CodeLocation
{
    public function __construct(
        public string $file_contents,
        string $file_path,
        public string $file_name,
        int $file_start,
        int $file_end
    ) {
        $this->file_path = $file_path;
        $this->file_start = $file_start;
        $this->file_end = $file_end;
    }
}
