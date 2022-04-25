import json
from pathlib import Path
import pandas as pd

# Use rustymind to parse and collect your own brainwave training data from the headset
directory = './train_data'
path_list = Path(directory).glob('**/*.txt')
data_list = []
count = 0

for path in path_list:
    for line in open(path, 'r'):
        data = json.loads(line)
        if data['poor_signal'] == 0 and data['attention'] != 0 and data['meditation'] != 0:   # attention or meditation equals 0 indicating low reliability
            if data['attention'] >= 1 and data['attention'] < 20:   # a value between 1 to 20 indicates "strongly lowered" levels
                data['attention_level'] = 0   # 0 represents "strongly lowered" level
            elif data['attention'] >= 20 and data['attention'] < 40:   #  a value between 20 to 40 indicates "reduced" levels
                data['attention_level'] = 1   # 1 represents "reduced" level
            elif data['attention'] >= 40 and data['attention'] < 60:   #  a value between 40 to 60 indicates "neutral" levels
                data['attention_level'] = 2   # 2 represents "neutral" level
            elif data['attention'] >= 60 and data['attention'] < 80:   #  a value between 60 to 80 indicates "slightly elevated" levels
                data['attention_level'] = 3   # 3 represents "slightly elevated" level
            elif data['attention'] >= 80 and data['attention'] <= 100:   #  a value between 80 to 100 indicates "elevated" levels
                data['attention_level'] = 4   # 4 represents "elevated" level
            else:
                print('attention level out of range', data['attention'])
            if data['meditation'] >= 1 and data['meditation'] < 20:   # a value between 1 to 20 indicates "strongly lowered" levels
                data['meditation_level'] = 0   # 0 represents "strongly lowered" level
            elif data['meditation'] >= 20 and data['meditation'] < 40:   #  a value between 20 to 40 indicates "reduced" levels
                data['meditation_level'] = 1   # 1 represents "reduced" level
            elif data['meditation'] >= 40 and data['meditation'] < 60:   #  a value between 40 to 60 indicates "neutral" levels
                data['meditation_level'] = 2   # 2 represents "neutral" level
            elif data['meditation'] >= 60 and data['meditation'] < 80:   #  a value between 60 to 80 indicates "slightly elevated" levels
                data['meditation_level'] = 3   # 3 represents "slightly elevated" level
            elif data['meditation'] >= 80 and data['meditation'] <= 100:   #  a value between 80 to 100 indicates "elevated" levels
                data['meditation_level'] = 4   # 4 represents "elevated" level
            else:
                print('meditation level out of range', data['meditation'])
            data['delta'] = data['eeg']['delta']
            data['theta'] = data['eeg']['theta']
            data['low_alpha'] = data['eeg']['low_alpha']
            data['high_alpha'] = data['eeg']['high_alpha']
            data['low_beta'] = data['eeg']['low_beta']
            data['high_beta'] = data['eeg']['high_beta']
            data['low_gamma'] = data['eeg']['low_gamma']
            data['mid_gamma'] = data['eeg']['mid_gamma']
            data_list.append(data)
            count += 1
print('number of samples=', count)


df = pd.DataFrame.from_records(data_list)
df.to_csv("./train_data/train_data_combined.csv", index=False)
print(df.head())

