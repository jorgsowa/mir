===description===
Constructor-promoted properties are not reported as UnusedParam
===file===
<?php
class Point {
    public function __construct(
        public readonly float $x,
        public readonly float $y,
        public readonly float $z = 0.0,
    ) {}
}

class Tagged {
    public function __construct(
        private string $name,
        protected int $priority = 0,
    ) {}
}
===expect===
