===description===
FN: trait constants never checked the #[Deprecated] attribute fallback,
unlike class constants — only the @deprecated docblock tag worked.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
trait OldHelpers {
    #[Deprecated]
    const LIMIT = 10;
}

class Service {
    use OldHelpers;
}

$v = Service::LIMIT;
//            ^^^^^ DeprecatedConstant: Constant Service::LIMIT is deprecated
