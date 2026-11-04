===description===
A mixed value assigned to a nullable property may still be null
===file===
<?php
final class C {
    private ?string $f = null;

    public function run(mixed $v): void {
        $this->f = $v;
        if (null === $this->f) {
            echo 'none';
        }
    }
}
