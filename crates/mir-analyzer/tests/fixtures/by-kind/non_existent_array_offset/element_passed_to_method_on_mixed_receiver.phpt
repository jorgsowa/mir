===description===
An array element passed to a method of an unknown (`mixed`) receiver may be written through a by-ref parameter, so its old shape no longer holds; an element passed to a known by-value method keeps it.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedPropertyFetch errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface FormState { public function get(string $key): mixed; }

function unknownReceiver(array $form, FormState $state): mixed {
    $handler = $state->get('handler');
    $form['options'] = ['#theme_wrappers' => ['container']];
    $handler->buildOptionsForm($form['options'], $state);
    return $form['options']['#title'];
}

final class Handler {
    public function inspect(array $options): int { return count($options); }
}

function knownReceiver(array $form, Handler $handler): mixed {
    $form['options'] = ['#theme_wrappers' => ['container']];
    $handler->inspect($form['options']);
    return $form['options']['#title'];
//                          ^^^^^^^^ NonExistentArrayOffset: Array offset '#title' does not exist
}
