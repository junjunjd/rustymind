import numpy as np
import pandas as pd
from sklearn.model_selection import train_test_split
import matplotlib.pyplot as plt
from sklearn import linear_model, metrics

f = './train_data/train_data_combined.csv'
df = pd.read_csv(f)
print(df.head())
print(list(df.columns))

X = df[['delta', 'theta', 'low_alpha', 'high_alpha', 'low_beta', 'high_beta', 'low_gamma', 'mid_gamma']]
y = df['attention']

X_train, X_test, y_train, y_test = train_test_split(
   X, y, test_size = 0.3, random_state = 1
)

regressor = linear_model.TweedieRegressor(power=1, alpha=0.5, link='log')
regressor.fit(X_train, y_train)
y_pred = regressor.predict(X_test)
df = pd.DataFrame({'Actual': y_test, 'Predicted': y_pred})
print(regressor.coef_)
print(regressor.intercept_)
print(df)
print('Mean Absolute Error:', metrics.mean_absolute_error(y_test, y_pred))
print('Mean Squared Error:', metrics.mean_squared_error(y_test, y_pred))
print('Root Mean Squared Error:', np.sqrt(metrics.mean_squared_error(y_test, y_pred)))
print('Explained variance score:', metrics.explained_variance_score(y_test, y_pred))
print('Max error:', metrics.max_error(y_test, y_pred))
print('r2_score:', metrics.r2_score(y_test, y_pred))
