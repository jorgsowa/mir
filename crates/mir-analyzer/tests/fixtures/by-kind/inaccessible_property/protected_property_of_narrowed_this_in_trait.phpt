===description===
Inside a trait, a protected property reached through a narrowed `$this` isn't flagged: the using class may extend the declaring class.
===file===
<?php
abstract class AbstractConfigurator {
    protected ?string $definition = null;
}
final class DefaultsConfigurator extends AbstractConfigurator {
    use BindTrait;
}
trait BindTrait {
    public function bind(): ?string {
        if ($this instanceof DefaultsConfigurator) {
            return $this->definition;
        }
        return null;
    }
}
