===description===
Class string properly resolves
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class ConfigRegistry {
    public function getValue() {
        return "config";
    }
}

/**
 * @param class-string<ConfigRegistry> $className
 */
function instantiate(string $className) {
    $instance = new $className();
    return $instance->getValue();
}

instantiate(ConfigRegistry::class);
