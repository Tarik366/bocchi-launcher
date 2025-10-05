# If you want to edit this code, please edit the file at:
# https://github.com/Tarik366/bocchi-launcher/blob/master/src/utilities/file.rs#L124

# extract_file(iso_path, file_path, output_path)
# iso_path: Path to the ISO file
# file_path: Path inside the ISO to list files from
# output_path: Path inside the ISO to extract
def extract_file(iso_path, file_path, output_path):
    import pycdlib
    from io import BytesIO
    from pathlib import Path
    iso = pycdlib.PyCdlib()
    iso.open(iso_path)

    # Create a BytesIO object to hold the extracted file content
    # And extract the file from the ISO to the BytesIO object
    extracted = BytesIO()
    iso.get_file_from_iso_fp(extracted, iso_path=file_path)
    Path(output_path.rsplit('\\', 1)[0]).mkdir(parents=True, exist_ok=True)
    # Write the extracted content to the output path
    with open(output_path, 'wb') as f:
        f.write(extracted.getvalue())

    iso.close()
    return output_path

extract_file("D:\\games\\psp\\Persona 2 Innocent Sin (USA) DLC enabler mod (v3)[Undub].iso", "/PSP_GAME/ICON0.PNG", "temp\\ULUS\\ICON0.PNG")