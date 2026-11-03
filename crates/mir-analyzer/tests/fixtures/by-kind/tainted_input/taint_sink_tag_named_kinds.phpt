===description===
`@taint-sink file|include|eval|unserialize` raise the same issue as the matching built-in sink.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @taint-sink file $f
 * @taint-sink include $i
 * @taint-sink eval $e
 * @taint-sink unserialize $u
 */
function sinks(string $f, string $i, string $e, string $u): void {
}

sinks((string) $_GET['f'], 'x', 'x', 'x');
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
sinks('x', (string) $_GET['i'], 'x', 'x');
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInclude: Tainted include path — possible file inclusion
sinks('x', 'x', (string) $_GET['e'], 'x');
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedEval: Tainted eval() code — possible code injection
sinks('x', 'x', 'x', (string) $_GET['u']);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedUnserialize: Tainted unserialize() payload — possible object injection
