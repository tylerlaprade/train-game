// CVC (consonant-vowel-consonant) words for early phonics. Avoid anything
// rude-sounding when a 2-year-old says it back loudly in public.
pub const WORDS: &[&str] = &[
    "cat", "dog", "sun", "bus", "car", "hat", "bat", "rat", "fox", "pig",
    "cow", "owl", "bee", "ant", "bug", "cup", "pot", "pan", "jam", "bag",
    "box", "mom", "dad", "kid", "pup", "tot", "toy", "hop", "run", "sit",
    "nap", "bed", "mug", "log", "mud", "tag", "bow", "fig", "ham", "jar",
    "key", "leg", "map", "net", "pen", "pin", "saw", "sky", "top", "van",
    "web", "yak", "zip", "ham", "fan", "ten", "six", "two",
];

pub fn random_word(rng: &mut impl rand::Rng) -> &'static str {
    use rand::seq::SliceRandom;
    WORDS.choose(rng).copied().unwrap_or("cat")
}
