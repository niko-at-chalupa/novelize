import json
from pathlib import Path
import ollama

# Adjust these to your local Ollama models
EXPENSIVE_MODEL = "llama3.3"  # or qwen2.5:32b / mistral-large
CHEAP_MODEL = "llama3.2"      # or qwen2.5:7b / phi4


def llm(model: str, prompt: str, system: str = "") -> str:
    messages = []
    if system:
        messages.append({"role": "system", "content": system})
    messages.append({"role": "user", "content": prompt})

    res = ollama.chat(model=model, messages=messages)
    return res["message"]["content"]


def run_pipeline(user_prompt: str, base_dir: Path):
    char_info = (base_dir / "data/character_info.txt").read_text()
    template_vn = (base_dir / "templates/template.rpy").read_text()

    # --- Step 1: Info Generation Step (Expensive Model) ---
    print("[1/5] Generating Topic and Story documents...")
    topic_doc = llm(
        EXPENSIVE_MODEL,
        f"Topic prompt: {user_prompt}\nCharacter Info:\n{char_info}",
        system="Generate a comprehensive educational topic document explaining key concepts.",
    )

    story_doc = llm(
        EXPENSIVE_MODEL,
        f"Character Info:\n{char_info}",
        system="Generate a lighthearted B-plot narrative script/ideas between these characters.",
    )

    # --- Step 2: Novelization Pass 1 - Topic (Cheap Model) ---
    print("[2/5] Pass 1: Generating main educational VN script...")
    pass1_vn = llm(
        CHEAP_MODEL,
        f"Topic Doc:\n{topic_doc}\n\nTemplate VN:\n{template_vn}\n\nCharacter Info:\n{char_info}",
        system="Convert the topic doc into an extensive Ren'Py (.rpy) script. Follow exact template formatting and character sprites.",
    )

    # --- Step 3: Novelization Pass 2 - B-Plot Insertion (Cheap Model) ---
    print("[3/5] Pass 2: Breaking story doc into scenes...")
    scenes_raw = llm(
        CHEAP_MODEL,
        f"Story Doc:\n{story_doc}",
        system="Break this story document into a JSON array of separate short scene descriptions. Output ONLY valid JSON array of strings.",
    )

    try:
        scenes = json.loads(scenes_raw)
        if not isinstance(scenes, list):
            scenes = [scenes_raw]
    except Exception:
        scenes = [scenes_raw]

    print("[3/5] Pass 2: Incorporating B-plot scenes into VN script...")
    current_vn = pass1_vn
    for i, scene in enumerate(scenes, 1):
        current_vn = llm(
            CHEAP_MODEL,
            f"Current VN:\n{current_vn}\n\nScene to insert:\n{scene}\n\nCharacter Info:\n{char_info}",
            system="Implement this B-plot scene into the Ren'Py script seamlessly while preserving existing teaching content.",
        )

    full_vn = current_vn

    # --- Step 4: Verification / Coherence Step (Expensive Model) ---
    print("[4/5] Verification Pass 1: Checking dialogue coherence...")
    coherent_vn = llm(
        EXPENSIVE_MODEL,
        f"Full VN:\n{full_vn}\n\nCharacter Info:\n{char_info}",
        system="Refine dialogue, character voices, sprite tags, and Ren'Py syntax to make it fully coherent.",
    )

    print("[5/5] Verification Pass 2: Fact-checking against topic document...")
    final_vn = llm(
        EXPENSIVE_MODEL,
        f"Coherent VN:\n{coherent_vn}\n\nOriginal Topic Document:\n{topic_doc}",
        system="Correct any factual errors or hallucinations in the script against the topic doc. Output ONLY valid Ren'Py code.",
    )

    output_path = base_dir / "output_script.rpy"
    output_path.write_text(final_vn)
    print(f"\nDone! Saved final Visual Novel to {output_path}")


def main():
    base_dir = Path(__file__).resolve().parent.parent.parent
    user_prompt = "How does concurrency work in Rust?"
    run_pipeline(user_prompt, base_dir)


if __name__ == "__main__":
    main()