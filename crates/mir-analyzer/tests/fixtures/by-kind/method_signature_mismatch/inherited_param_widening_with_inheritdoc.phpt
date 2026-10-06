===description===
`{@inheritDoc}` should inherit the parent docblock; parameter widening is contravariant-legal anyway.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
interface Manager {
    /** @param non-empty-list<int> $rows */
    public function rename(array $rows): void;
}
class ManagerImpl implements Manager {
    /**
     * {@inheritDoc}
     */
    public function rename(array $rows): void {}
}
