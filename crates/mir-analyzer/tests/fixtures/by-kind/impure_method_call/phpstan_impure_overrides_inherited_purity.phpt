===description===
An explicit @phpstan-impure declaration overrides a pure interface contract:
the implementation body is not analyzed as pure and pure callers still see
the method as impure.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Builder {
    /** @pure */
    public function build(): string;
}

final class LegacyBuilder implements Builder {
    /** @phpstan-impure */
    public function build(): string {
        return uniqid();
    }
}

/** @pure */
function wrap(LegacyBuilder $builder): string {
    return $builder->build();
//         ^^^^^^^^^^^^^^^^^ ImpureMethodCall: Calling impure method build() in a pure or immutable context
}
===expect===
