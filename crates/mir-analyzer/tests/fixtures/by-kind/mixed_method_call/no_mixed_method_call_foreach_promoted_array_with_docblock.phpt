===description===
No MixedMethodCall when foreach over promoted array property typed by @param docblock
===file===
<?php
interface Source {
    public function key(): string;
}

class ProjectSearch {
    /**
     * @param array<int, Source> $sources
     */
    public function __construct(
        private readonly array $sources
    ) {}

    public function run(): void {
        foreach ($this->sources as $source) {
            $source->key();
        }
    }
}
===expect===
