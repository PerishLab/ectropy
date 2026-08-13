mod seat;
use seat::*;

#[test]
fn structure() {
    let source = r#"<script lang="ts">
        let count = $state(0);
        let doubled = $derived(count * 2);
        function raise(): void { count += 1; }
        $: legacy = doubled + 1;
    </script>
    <!-- a component comment -->
    {#if count > 0}
        <button class:active={count > 2} on:click={raise}>{doubled}</button>
    {:else if legacy > 0}
        <span>legacy</span>
    {:else}
        <input bind:value={count} />
    {/if}
    {#each [count] as item}<p>{item}</p>{:else}<p>empty</p>{/each}
    {#await Promise.resolve(count)}<p>wait</p>{:then value}<p>{value}</p>{:catch fault}<p>{fault}</p>{/await}
    {#snippet row(value)}<strong>{value}</strong>{/snippet}
    {@render row(count)}
    <style>.active { color: red; }</style>"#;
    let found = scan("Counter.svelte", source);
    assert!(!found.contains(&"coverage".to_string()), "{found:?}");
    assert!(!found.contains(&"word".to_string()), "{found:?}");
    assert!(found.contains(&"comment".to_string()));
}

#[test]
fn style() {
    let mut config = config(&[], 4);
    config.file.grant.push(kernel::config::Grant {
        syntax: "style".to_string(),
        paths: vec!["styles/**".to_string()],
    });
    let source = "<script lang=\"ts\">let on = true;</script><main class:active={on}>x</main><style>.active { color: red; }</style>";
    assert_eq!(hits("src/Card.svelte", source, &config), 1);
    assert_eq!(hits("styles/Card.svelte", source, &config), 0);
}
