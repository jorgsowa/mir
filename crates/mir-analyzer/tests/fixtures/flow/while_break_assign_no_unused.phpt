===description===
Variable assigned inside while-condition via ($x = expr()) === null pattern and consumed
in the loop body must not be reported as unused — across multiple fixpoint iterations.
===config===
<mir>
  <issueHandlers>
    <PossiblyNullMethodCall errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Ex {
    public function getClass(): string;
    /** @return static|null */
    public function getPrevious(): ?self;
}

function test(Ex $exception): void {
    while ($exception->getClass() === 'Foo') {
        if (($previous = $exception->getPrevious()) === null) {
            break;
        }
        $exception = $previous;
    }
}
===expect===
