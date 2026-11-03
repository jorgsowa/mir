===description===
An alias may name a method the aliased trait gets from a trait it uses itself.
===file===
<?php
trait CommonTrait {
    protected function doClear(string $ns): bool { return $ns !== ''; }
}
trait FsTrait {
    use CommonTrait;
}
final class Adapter {
    use FsTrait {
        doClear as private doClearCache;
    }
    public function clear(): bool { return $this->doClearCache('x'); }
}
