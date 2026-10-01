<?php
namespace Psalm;

use Psalm\Internal\Provider\FileReferenceProvider;
use Psalm\Storage\ClassLikeStorage;

class Codebase
{
    public bool $collect_references = false;
    public object $classlikes;
    public object $scanner;
    public object $classlike_storage_provider;
    public FileReferenceProvider $file_reference_provider;
    /** @var list<string> */
    public array $scanned = [];

    public function __construct(public Config $config)
    {
        $codebase = $this;
        $this->classlikes = new class {
            public bool $collect_references = false;
        };
        $this->scanner = new class($codebase) {
            public function __construct(private Codebase $codebase)
            {
            }

            public function addFileToDeepScan(string $file): void
            {
                $this->codebase->scanned[] = $file;
            }
        };
        $this->classlike_storage_provider = new class {
            /** @var array<string, ClassLikeStorage> */
            private array $storages = [];

            public function get(string $fqcn): ClassLikeStorage
            {
                return $this->storages[strtolower($fqcn)] ??= new ClassLikeStorage($fqcn);
            }
        };
        $this->file_reference_provider = new FileReferenceProvider();
    }

    public function scanFiles(): void
    {
    }

    public function queueClassLikeForScanning(string $fqcn): void
    {
    }
}
