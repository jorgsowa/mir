===description===
A class unrelated to a method template's bound is an invalid argument, on instance and static calls.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Element {}
class Comment {}

final class Document {
    /**
     * @template T of Element
     * @param T $node
     */
    public function adopt(object $node): void {}

    /**
     * @template T of Element
     * @param T $node
     */
    public static function register(object $node): void {}
}

function run(Document $doc, Comment $comment): void {
    $doc->adopt($comment);
//  ^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $node of adopt() expects 'Element', got 'Comment'
    Document::register($comment);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $node of register() expects 'Element', got 'Comment'
}
