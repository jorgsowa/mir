===description===
A trace frame's `type` is `'->'` or `'::'`, on Throwable, Exception and Error alike.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @psalm-type Frame = array{function: string, line?: int, file?: string, class?: class-string, type?: '->'|'::', args?: list<mixed>, object?: object} */
final class TraceHolder {
    /** @var list<Frame> */
    private array $trace;

    public function __construct(\Throwable $t, \Exception $e, \Error $err) {
        $this->trace = $t->getTrace();
        $this->trace = $e->getTrace();
        $this->trace = $err->getTrace();
        foreach ($e->getTrace() as $frame) {
            $type = $frame['type'] ?? '->';
            /** @mir-check $type is "->"|"::" */
            echo $type;
        }
    }
}
