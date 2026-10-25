"""Read slide order separately from stable source filenames."""
import json

def load_deck(here):
    layout = json.loads((here / 'review-layout.json').read_text())
    slides = []
    for number, source in enumerate(layout['sources'], 1):
        slide = json.loads((here / source).read_text())
        slide.update(slide=number, source_file=source)
        slides.append(slide)
    assert len(slides) == layout['count']
    return slides, layout
