===description===
Only scalar and plain-object shape values are judged: a template class-string and a wider list are left alone.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Model {}

/** @template TModel of Model */
final class Info {
    /** @param class-string<TModel> $class */
    public function __construct(public string $class) {}

    /** @return array{class: class-string<Model>} */
    public function toArray(): array {
        return ['class' => $this->class];
    }
}

/**
 * @param array<int|string, bool> $found
 * @return array{names: array<string>}
 */
function names(array $found): array {
    return ['names' => array_keys($found)];
}
