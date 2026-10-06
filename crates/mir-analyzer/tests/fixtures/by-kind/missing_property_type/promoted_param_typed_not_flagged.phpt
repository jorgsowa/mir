===description===
MissingPropertyType does NOT fire for promoted constructor parameters that have a type declaration.
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
class Point {
    public function __construct(
        public float $x,
        public float $y,
        protected int $count = 0,
        private ?string $label = null,
    ) {}
}
